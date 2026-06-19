<details>
<summary>XSD contract: <code>DataflowType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="DataflowType">
		<xs:annotation>
			<xs:documentation>DataflowType describes the structure of a data flow. Using a DimensionConstraint and/or a DataConstraint a data flow can define a subset of data defined by a DataStructure. Unless the dataflow artefact is defined externally, a reference to a DataStructure must be provided.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="DataflowBaseType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element ref="common:Name" maxOccurs="unbounded"/>
					<xs:element ref="common:Description" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="Structure" type="common:DataStructureReferenceType" minOccurs="0">
						<xs:annotation>
							<xs:documentation>Structure provides a reference to the data structure definition that frames the underlying structure of the data defined this data flow.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="DimensionConstraint" type="DimensionConstraintType" minOccurs="0" maxOccurs="1">
						<xs:annotation>
							<xs:documentation>Required if the DataStructure defines itself as an evolving structure, indicating that it can change dimensionality under a minor version change, and if the Dataflow references that DataStructure using a wildcarded minor version number. New minor DSD version can so still be used by this Dataflow even if that DSD version defines new additional Dimensions.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
				<xs:attribute name="urn" type="common:DataflowUrnType" use="optional"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
