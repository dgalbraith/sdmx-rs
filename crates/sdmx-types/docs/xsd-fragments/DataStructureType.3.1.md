<details>
<summary>XSD contract: <code>DataStructureType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="DataStructureType">
		<xs:annotation>
			<xs:documentation>DataStructureType defines the structure for a data structure definition. A data structure definition is defined as a collection of metadata concepts, their structure and usage when used to collect or disseminate data.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="DataStructureBaseType">
				<xs:sequence>
					<xs:element name="Metadata" type="common:MetadataStructureReferenceType" minOccurs="0">
						<xs:annotation>
							<xs:documentation>A data structure definition may be related to a metadata structure definition in order to use its metadata attributes as part of the data. Note that the referenced metadata set cannot contain nested metadata attributes, as these are not supported in the data. By default all metadata attributes can be associated at any level of the data. However, a metadata attribute usage can be used to provide a specific attribute relationship for a given metadata attribute.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
				<xs:attribute name="evolvingStructure" type="xs:boolean" use="optional" default="false">
					<xs:annotation>
						<xs:documentation>An evolving data structure indicates that new Dimensions may be added under a minor version update e.g. 1.0.0 to 1.1.0. Evolving Data Structures can only be used by Dataflows if they include a DimensionConstraint to fix the Dimensions to the subset required by the Dataflow.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
