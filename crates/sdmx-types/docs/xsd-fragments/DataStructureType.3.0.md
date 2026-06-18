<details>
<summary>XSD contract: <code>DataStructureType</code> (SDMX 3.0)</summary>

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
							<xs:documentation>A data structure definition may be related to a metadata structure definition in order to use its metadata attributes as part of the data. Note that the referenced metadata set cannot contain nested metadata attributes, as these are not supported in the data. By default all metadata attributes can be associated at any level of the data. However, a metadata attribute usage can be used to provide a specific attribute relationshp for a given metadata attribute.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
